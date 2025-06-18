.PHONY: cert/generate
cert/generate:
	mkcert -key-file self_signed_certs/key.pem -cert-file self_signed_certs/cert.pem localhost 127.0.0.1 ::1
