package main

import (
	"context"
	"encoding/json"
	"fmt"
	"github.com/coder/websocket"
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

func NewNapCat(ctx context.Context, url string) *NapCat {
	ws := NewWebSocket(ctx, url)
	return &NapCat{
		ws: ws,
	}
}

type NapCatRequest struct {
	Action string         `json:"action"`
	Params map[string]any `json:"params"`
	Echo   string         `json:"echo"`
}

func (c *NapCat) send(groupID, userID int, rawMessage string) {
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
		return
	}
	c.ws.Write(ctx, websocket.MessageText, jsonData)
}

func (c *NapCat) recv(resp *NapCatResponse) error {
	_, jsonData := c.ws.Read(context.Background())
	if err := json.Unmarshal(jsonData, resp); err != nil {
		return err
	}
	return nil
}
