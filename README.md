# Gradecoin

This will sit behind nginx reverse proxy so running at 127.0.0.1:8080 is no problem, or https.

```
$ cargo run

$ curl --location --request POST 'localhost:8080/transaction' --header 'Content-Type: application/json' --data-raw '{
  "source": "Myself Truly",
  "target": "Literally Anybody Else",
  "amount": 12,
  "timestamp": "2021-04-07T00:17:00"
}'
```

