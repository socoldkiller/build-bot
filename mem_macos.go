//go:build darwin
// +build darwin

package main

func memfdCreate(name string) (string, error) {
	return name, nil
}
