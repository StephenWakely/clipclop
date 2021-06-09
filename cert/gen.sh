#!/bin/sh

# from https://dev.to/techschoolguru/how-to-secure-grpc-connection-with-ssl-tls-in-go-4ph

rm *.pem

# 1. Generate CA's private key and self-signed certificate
openssl req -x509 -newkey rsa:4096 -days 365 -nodes -keyout ca-key.pem -out ca-cert.pem -subj "/C=GB/ST=Thing/L=Morething/O=NotherThing/OU=Education/CN=*.local/emailAddress=thing@thang.com"

echo "CA's self-signed certificate"
openssl x509 -in ca-cert.pem -noout -text

# 2. Generate web server's private key and certificate signing request (CSR)
openssl req -newkey rsa:4096 -nodes -keyout machine1-key.pem -out machine1-req.pem -subj "/C=GB/ST=Thing/L=Thang/O=Thong/OU=Computer/CN=*.local/emailAddress=thong@thing.com"

# 3. Use CA's private key to sign web server's CSR and get back the signed certificate
openssl x509 -req -in machine1-req.pem -days 60 -CA ca-cert.pem -CAkey ca-key.pem -CAcreateserial -out machine1-cert.pem -extfile ext.cnf

echo "Machine 1's signed certificate"
openssl x509 -in machine1-cert.pem -noout -text

# 4. Generate client's private key and certificate signing request (CSR)
openssl req -newkey rsa:4096 -nodes -keyout machine2-key.pem -out machine2-req.pem -subj "/C=GB/ST=Thing/L=Thang/O=Thung/OU=Computer/CN=*.local/emailAddress=thung@theng.com"

# 5. Use CA's private key to sign client's CSR and get back the signed certificate
openssl x509 -req -in machine2-req.pem -days 60 -CA ca-cert.pem -CAkey ca-key.pem -CAcreateserial -out machine2-cert.pem -extfile ext.cnf

echo "Machine 2's signed certificate"
openssl x509 -in machine2-cert.pem -noout -text
