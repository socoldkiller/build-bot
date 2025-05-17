package main

import "sync"

type MsgQueue struct {
	q sync.Map
}

func (m *MsgQueue) Store(k Session, v chan *NapCatResponse) {
	m.q.Store(k, v)
}

func (m *MsgQueue) Load(k Session) (chan *NapCatResponse, bool) {
	r, ok := m.q.Load(k)
	if !ok {
		return nil, ok
	}
	return r.(chan *NapCatResponse), ok
}

func (m *MsgQueue) Delete(k Session) {
	m.q.Delete(k)
}
