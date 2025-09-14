package main

import (
	"bytes"
	"context"
	"errors"
	"github.com/sirupsen/logrus"
	"io"
)

type DelimitedReader struct {
	r io.Reader
}

func NewDelimitedReader(r io.Reader) *DelimitedReader {
	return &DelimitedReader{
		r: r,
	}
}

func (dr *DelimitedReader) Read(p []byte) (int, error) {
	return dr.r.Read(p)
}

func (dr *DelimitedReader) ReadString(ctx context.Context, delim []byte) (string, error) {
	var buf []byte
	b := make([]byte, 1024)

	go func() {
		<-ctx.Done()

		switch {
		case errors.Is(ctx.Err(), context.DeadlineExceeded):
			if closer, ok := dr.r.(io.Closer); ok {
				_ = closer.Close()
				return
			}

			logrus.Warnf("If you don't implement the closer interface, you may not be able to cancel the read operation correctly")
		case errors.Is(ctx.Err(), context.Canceled):
			//TODO
			return

		default:
			return
		}
	}()

	for {
		if idx := bytes.Index(buf, delim); idx != -1 {
			return string(buf[:idx]), nil
		}
		n, err := dr.Read(b)
		if n > 0 {
			buf = append(buf, b[:n]...)
		}
		switch err {
		case io.EOF:
			return string(buf), nil
		case nil:
			continue
		default:
			return string(buf), err
		}
	}
}
