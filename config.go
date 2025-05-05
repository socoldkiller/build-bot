package main

import (
	"github.com/sirupsen/logrus"
	"gopkg.in/yaml.v3"
	"os"
)

type Config struct {
	URL    string `yaml:"url"`
	Prefix string `yaml:"prefix"`
}

var GlobalCfg *Config

func init() {
	GlobalCfg = new(Config)
	cfg, _ := os.Open("config.yaml")
	err := yaml.NewDecoder(cfg).Decode(GlobalCfg)
	if err != nil {
		logrus.Fatalf("parse config.yaml error")
	}

}
