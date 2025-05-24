package main

import (
	"context"
	"encoding/json"
	"fmt"
	"github.com/coder/websocket"
	"github.com/sirupsen/logrus"
	"io"
	"os/exec"
	"strings"
)

func judgeOutput(err error, stdout string, stderr string) string {
	output := fmt.Sprintf("%s\n%s", stdout, stderr)
	output = strings.TrimSpace(output)
	if output == "" {
		output = err.Error()
	}
	return output
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
	ws *Websocket
}

func NewNapCat(ws *Websocket) *NapCat {
	return &NapCat{
		ws: ws,
	}
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
	jsonData, err := json.Marshal(body)
	if err != nil {
		logrus.Warnf("body(%s) can't marshal,err %s", jsonData, err)
		return nil
	}
	return c.ws.Write(ctx, websocket.MessageText, jsonData)
}

func (c *NapCat) recv(resp *NapCatResponse) error {
	_, jsonData, err := c.ws.Read(context.Background())
	if err != nil {
		return err
	}

	if err := json.Unmarshal(jsonData, resp); err != nil {
		return nil
	}
	return nil
}
