package main

import (
	"bufio"
	"fmt"
	"net"
)

func handleConnection(conn net.Conn) {
	defer conn.Close()
	addr := conn.RemoteAddr().String()
	fmt.Println("Connected to:", addr)

	scanner := bufio.NewScanner(conn)
	for scanner.Scan() {
		text := scanner.Text()
		fmt.Printf("Received from %s: %s\n", addr, text)
		// Echo back
		fmt.Fprintf(conn, "Echo: %s\n", text)
	}
	fmt.Println("Disconnected:", addr)
}

func main() {
	listener, err := net.Listen("tcp", ":4000")
	if err != nil {
		panic(err)
	}
	defer listener.Close()
	fmt.Println("Listening on :4000")

	for {
		conn, err := listener.Accept()
		if err != nil {
			fmt.Println("Failed to accept:", err)
			continue
		}
		go handleConnection(conn) // handle each connection in a new goroutine
	}
}
