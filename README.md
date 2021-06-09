ClipClop
========

Keep the clipboard of two machines in sync.

To run, first generate certificates. Edit `cert/ext.ctf` and replace machine1 and machine2 with the actual host names of the machines you want to keep in sync. Then run `cert/gen.sh`:


On machine 1 run:

```
> clipclop --server https://machine2.local:9998 --cacert cert/ca-cert.pem --cert cert/machine1-cert.pem --key cert/machine1-key.pem
```

Securely copy `machine2-cert.pem`, `machine2-key.pem` and `ca-cert.pem` to machine 2. Then run: 

```
> clipclop --server https://machine1.local:9998 --cacert cert/ca-cert.pem --cert cert/machine2-cert.pem --key cert/machine2-key.pem
```


Any text (only works with text) you copy on one machine will be sent to the clipboard of the other machine.


