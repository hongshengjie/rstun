package main

import (
	"fmt"

	"gortc.io/stun"
)

func main() {
	fetch()
}

func fetch() {
retry:
	c, err := stun.Dial("udp", "127.0.0.1:3478")
	if err != nil {
		panic(err)
	}
	message := stun.MustBuild(stun.TransactionID, stun.BindingRequest)
	if err := c.Do(message, func(res stun.Event) {
		if res.Error != nil {
			panic(res.Error)
		}

		var xorAddr stun.XORMappedAddress
		if err := xorAddr.GetFrom(res.Message); err != nil {
			panic(err)
		}
		fmt.Println("your IP is", xorAddr.IP, xorAddr.Port)
	}); err != nil {
		panic(err)
	}
	c.Close()
	goto retry
}
