package main

import (
	"bytes"
	"io"
)

type DelimitedReader struct {
	r      io.Reader
	delim  []byte
	buffer []byte
	eofHit bool
}

func NewDelimitedReader(r io.Reader, delim string) *DelimitedReader {
	return &DelimitedReader{
		r:     r,
		delim: []byte(delim),
	}
}

func (dr *DelimitedReader) Read(p []byte) (int, error) {
	if dr.eofHit {
		dr.eofHit = false
	}
	for {
		if idx := bytes.Index(dr.buffer, dr.delim); idx != -1 {
			n := copy(p, dr.buffer[:idx])
			dr.buffer = dr.buffer[idx+len(dr.delim):]
			dr.eofHit = true
			return n, io.EOF
		}

		tmp := make([]byte, 1024)
		n, err := dr.r.Read(tmp)
		if n > 0 {
			dr.buffer = append(dr.buffer, tmp[:n]...)
		}

		if err != nil {
			if len(dr.buffer) > 0 {
				n := copy(p, dr.buffer)
				dr.buffer = nil
				return n, io.EOF
			}
			return 0, err
		}
	}
}
