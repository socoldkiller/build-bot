package main

import (
	"context"
	"crypto/rand"
	"encoding/base64"
	"errors"
	"fmt"
	"io"
	"os/exec"
	"strings"
	"syscall"
	"time"

	"github.com/samber/lo"
	"github.com/sirupsen/logrus"
)

func CombineOutput(stdout, stderr string) string {
	output := fmt.Sprintf("%s\n%s", stdout, stderr)
	output = strings.TrimSpace(output)
	return output
}

type Shell struct {
	delim     []byte
	stdout    *DelimitedReader
	stderr    *DelimitedReader
	stdin     io.Writer
	closers   []io.Closer
	shell     *exec.Cmd
	shellType string
}

func NewShell(shellType string) (*Shell, error) {

	generateRandomDelimiter := func() []byte {
		b := make([]byte, 24)
		_, err := rand.Read(b)
		if err != nil {
			panic(err)
		}
		return []byte(base64.RawURLEncoding.EncodeToString(b))
	}

	delim := generateRandomDelimiter()
	stdoutReader, stdoutWriter := io.Pipe()
	stderrReader, stderrWriter := io.Pipe()
	outReader := NewDelimitedReader(stdoutReader)
	errReader := NewDelimitedReader(stderrReader)

	execShell := exec.Command(shellType)
	stdin, err := execShell.StdinPipe()
	execShell.Stdout = stdoutWriter
	execShell.Stderr = stderrWriter
	if err != nil {
		return nil, err
	}

	s := &Shell{
		delim:     delim,
		stdin:     stdin,
		stdout:    outReader,
		stderr:    errReader,
		closers:   []io.Closer{stdin},
		shell:     execShell,
		shellType: shellType,
	}
	startedChan := make(chan error)
	go waitShellStart(startedChan, s)
	if err = <-startedChan; err != nil {
		return nil, err
	}
	return s, nil
}

func waitShellStart(startedChan chan<- error, shell *Shell) {
	defer shell.Close()
	err := shell.shell.Start()
	startedChan <- err
	if err = shell.shell.Wait(); err != nil {
		logrus.Infof("%s wait: %v", shell.shellType, err)
	}
}

func (s *Shell) Close() error {
	errs := lo.Map(s.closers, func(closer io.Closer, _ int) error {
		return closer.Close()
	})
	return errors.Join(errs...)
}

func (s *Shell) Exec(ctx context.Context, cmd string) (map[string]string, error) {
	fullCmd := fmt.Sprintf("%s; echo %s; echo %s 1>&2\n", cmd, s.delim, s.delim)
	logrus.Debugf("full cmd '%s' ", fullCmd[:len(fullCmd)-1])
	if _, err := io.WriteString(s.stdin, fullCmd); err != nil {
		logrus.Warnf("can't write stdin command,err: %s", err)
		return nil, err
	}

	type result struct {
		out string
		err error
	}

	readResult := func(reader *DelimitedReader) result {
		out, err := reader.ReadString(s.delim)
		return result{out: out, err: err}
	}

	asyncRead := func() <-chan []result {
		var list []result
		resChan := make(chan []result)
		go func() {
			list = append(list, readResult(s.stdout))
			list = append(list, readResult(s.stderr))
			resChan <- list
		}()
		return resChan
	}

	select {
	case <-ctx.Done():
		return nil, ctx.Err()
	case list := <-asyncRead():
		var stdout, stderr result
		stdout = list[0]
		stderr = list[1]
		err := errors.Join(stdout.err, stderr.err)
		return map[string]string{
			"stdout": stdout.out,
			"stderr": stderr.out,
		}, err

	}

}

func TTyShell(ctx context.Context, shell *Shell, cat *NapCat, msgChan <-chan *NapCatResponse, stopChan chan<- error) {
	killCmd := func() {
		defer func() { stopChan <- nil }()
		var err error
		if err = shell.shell.Process.Signal(syscall.SIGTERM); err == nil {
			logrus.Infof("kill shell %s success,pid %d .", shell.shellType, shell.shell.Process.Pid)
			return
		}

		waitTimeout := 2 * time.Second
		time.Sleep(waitTimeout)
		if err = shell.shell.Process.Kill(); err != nil {
			logrus.Warnf("kill shell %s failed,pid %d err: %v", shell.shellType, shell.shell.Process.Pid, err)
		}

	}

	defer killCmd()
	for {
		select {
		case <-ctx.Done():
			logrus.Infof("TTyShell context canceled: %v", ctx.Err())
			return

		case <-time.After(5 * time.Minute):
			logrus.Infof("tty shell timeout")
			return

		case msg, ok := <-msgChan:
			if !ok {
				return
			}
			cmd := msg.Message[0].Data.Text

			shellExec := func(cmd string) (map[string]string, error) {
				ctx, cancel := context.WithTimeout(context.Background(), 5*time.Minute)
				defer cancel()
				return shell.Exec(ctx, cmd)
			}

			output, err := shellExec(cmd)

			switch {
			case errors.Is(err, context.DeadlineExceeded):
				cat.send(msg.GroupID, msg.UserID, err.Error())
				return
			case !errors.Is(err, nil):
				err := cat.send(msg.GroupID, msg.UserID, err.Error())
				if err != nil {
					return
				}

				continue
			}
			resp := CombineOutput(output["stdout"], output["stderr"])
			if resp == "" {
				continue
			}

			err = cat.send(msg.GroupID, msg.UserID, resp)
			if err != nil {
				return
			}
		}
	}

}
