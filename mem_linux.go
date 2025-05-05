//go:build linux
// +build linux

package main

import (
	"fmt"
	"syscall"
	"unsafe"
)

const SYS_MEMFD_CREATE = 319

func memfdCreate(name string) (string, error) {
	nameBytes, err := syscall.BytePtrFromString(name)
	if err != nil {
		return "", err
	}
	fd, _, errno := syscall.Syscall(SYS_MEMFD_CREATE, uintptr(unsafe.Pointer(nameBytes)), 0, 0)
	if int(fd) == -1 {
		return "", errno
	}
	return fmt.Sprintf("/proc/self/fd/%d", fd), nil
}
