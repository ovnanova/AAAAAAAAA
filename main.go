package main

import (
	"fmt"
	"math/rand"
	"os"
	"os/signal"
	"strings"
	"syscall"
	"time"
)

var charSet = []string{
	"A̵̦̦̓͌͗͛̕", "A", "₳", "░A░", "A҉", "Ⱥ", "A̷", "A̲", "A̳", "A̾",
	"A͎", "A͓̽", "𝔸", "ᴀ", "∀",
}

func randomString() string {
	length := rand.Intn(20) + 1
	var sb strings.Builder
	for i := 0; i < length; i++ {
		sb.WriteString(charSet[rand.Intn(len(charSet))])
	}
	return sb.String()
}

func main() {
	sigs := make(chan os.Signal, 1)
	signal.Notify(sigs, syscall.SIGINT, syscall.SIGTERM)

	go func() {
		for {
			fmt.Println(randomString())
			time.Sleep(100 * time.Millisecond)
		}
	}()

	<-sigs
}
