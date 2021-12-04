package main

import (
	"flag"
	"fmt"
	"log"
	"net"
	"strconv"
	"sync"

	"github.com/panjf2000/gnet"

	"gortc.io/stun"
	"gortc.io/turn"
)

type Service struct {
	*gnet.EventServer
	pool sync.Pool
}

func (es *Service) OnInitComplete(srv gnet.Server) (action gnet.Action) {
	log.Printf("UDP Stun server is listening on %s (multi-cores: %t, loops: %d)\n", srv.Addr.String(), srv.Multicore, srv.NumEventLoop)
	return
}

func (es *Service) React(frame []byte, c gnet.Conn) (out []byte, action gnet.Action) {
	if stun.IsMessage(frame) {
		req := es.pool.Get().(*stun.Message)
		req.Reset()

		req.Raw = frame
		err := req.Decode()
		if err != nil {
			return
		}
		if req.Type == stun.BindingRequest {
			host, port, err := net.SplitHostPort(c.RemoteAddr().String())
			if err != nil {
				return
			}
			porti, err := strconv.Atoi(port)
			if err != nil {
				return
			}

			client := &turn.Addr{IP: net.ParseIP(host), Port: porti}
			addr := (*stun.XORMappedAddress)(client)
			resp := es.pool.Get().(*stun.Message)
			resp.Reset()
			resp.Type = stun.MessageType{
				Class:  stun.ClassSuccessResponse,
				Method: req.Type.Method,
			}
			resp.TransactionID = req.TransactionID
			resp.WriteHeader()
			addr.AddTo(resp)
			stun.Fingerprint.AddTo(resp)
			out = resp.Raw

			es.pool.Put(req)
			es.pool.Put(resp)
			return
		}
	}
	return
}

func main() {
	var port int
	flag.IntVar(&port, "port", 3479, "--port 3479")
	flag.Parse()
	stun := &Service{
		pool: sync.Pool{
			New: func() interface{} {
				return new(stun.Message)
			},
		},
	}
	log.Fatal(gnet.Serve(stun, fmt.Sprintf("udp://:%d", port), gnet.WithMulticore(true), gnet.WithReusePort(true)))
}
