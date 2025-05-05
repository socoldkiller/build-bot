package main

import (
	"context"
	"encoding/json"
	"github.com/coder/websocket"
	"github.com/coder/websocket/wsjson"
	"io"
	"os/exec"
)

func judgeOutput(err error, stdout string, stderr string) string {
	if stdout != "" {
		return stdout
	}
	if stderr != "" {
		return stderr
	}

	if err != nil {
		return err.Error()
	}

	return ""
}

type CmdRunner struct {
	cmd  string
	args []string
}

func (runner *CmdRunner) Run(stdin io.Reader, stdout, stderr io.Writer) error {
	localCommand := exec.Command(runner.cmd, runner.args...)
	localCommand.Stdin = stdin
	localCommand.Stdout = stdout
	localCommand.Stderr = stderr
	err := localCommand.Run()
	return err
}

type NapCat struct {
	conn *websocket.Conn
}

type NapCatRequest struct {
	Action string         `json:"action"`
	Params map[string]any `json:"params"`
	Echo   string         `json:"echo"`
}

func (c *NapCat) send(groupID, userID int, rawMessage string) error {
	ctx := context.Background()
	var action string
	params := make(map[string]any)

	if groupID != 0 {
		params["group_id"] = groupID
		action = "send_group_msg"
	} else {
		action = "send_private_msg"
	}
	params["user_id"] = userID
	params["message"] = rawMessage

	body := &NapCatRequest{
		Action: action,
		Params: params,
	}

	return wsjson.Write(ctx, c.conn, body)
}

func (c *NapCat) recv(resp *NapCatResponse) error {
	ctx := context.Background()
	_, jsonData, err := c.conn.Read(ctx)
	if err != nil {
		return err
	}

	if err = json.Unmarshal(jsonData, resp); err != nil {
		return err
	}
	return nil
}
