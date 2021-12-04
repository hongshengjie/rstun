package main

import (
	"fmt"
	"sync"

	"gortc.io/stun"
)

func main() {
	var wg sync.WaitGroup
	for i := 0; i <= 1000; i++ {
		wg.Add(1)
		go func() {
			defer wg.Done()
			fetch()
		}()
	}
	wg.Wait()
}

func fetch() {
retry:
	// Creating a "connection" to STUN server.
	c, err := stun.Dial("udp", "127.0.0.1:3478")
	if err != nil {
		panic(err)
	}
	// Building binding request with random transaction id.
	message := stun.MustBuild(stun.TransactionID, stun.BindingRequest)
	// Sending request to STUN server, waiting for response message.
	if err := c.Do(message, func(res stun.Event) {
		if res.Error != nil {
			panic(res.Error)
		}
		// Decoding XOR-MAPPED-ADDRESS attribute from message.
		var xorAddr stun.XORMappedAddress
		if err := xorAddr.GetFrom(res.Message); err != nil {
			panic(err)
		}
		fmt.Println("your IP is", xorAddr.IP, xorAddr.Port)
		//fmt.Println()
	}); err != nil {
		panic(err)
	}
	c.Close()
	goto retry
}
