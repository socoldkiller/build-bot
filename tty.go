package main

import (
	"context"
	"errors"
	"fmt"
	"github.com/samber/lo"
	"github.com/sirupsen/logrus"
	"io"
	"os/exec"
	"strings"
)

func CombineOutput(stdout, stderr string) string {
	output := fmt.Sprintf("%s\n%s", stdout, stderr)
	output = strings.TrimSpace(output)
	return output
}

type Shell struct {
	delim   []byte
	stdout  *DelimitedReader
	stderr  *DelimitedReader
	stdin   io.Writer
	closers []io.Closer

	c *exec.Cmd
}

func NewShell(shellType string) (*Shell, error) {
	delim := []byte("__CMD_DONE__")
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
		delim:   delim,
		stdin:   stdin,
		stdout:  outReader,
		stderr:  errReader,
		closers: []io.Closer{stdin},
		c:       execShell,
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
	err := shell.c.Start()
	startedChan <- err
	if err = shell.c.Wait(); err != nil {
		logrus.Infof("%s wait: %s", shell, err)
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
	io.WriteString(s.stdin, fullCmd)

	stdout, err1 := s.stdout.ReadString(s.delim)
	stderr, err2 := s.stderr.ReadString(s.delim)
	err := errors.Join(err1, err2)
	return map[string]string{
		"stdout": stdout,
		"stderr": stderr,
	}, err
}

func TTyShell(shell *Shell, cat *NapCat, msgChan <-chan *NapCatResponse) {
	for msg := range msgChan {
		cmd := msg.Message[0].Data.Text
		output, err := shell.Exec(context.Background(), cmd)
		if err != nil {
			cat.send(msg.GroupID, msg.UserID, err.Error())
			continue
		}
		resp := CombineOutput(output["stdout"], output["stderr"])
		cat.send(msg.GroupID, msg.UserID, resp)

	}

}
