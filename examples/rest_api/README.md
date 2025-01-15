## REST API Server

Endpoints could be added by URL:

```shell
$ curl -s -X GET http://localhost:3000/endpoints | jq .
[]
$ curl -s -X POST --json '{"url":"http://api.example.com","state":"Connected"}' http://localhost:3000/endpoints | jq .
{
  "id": 0,
  "url": "http://api.example.com",
  "state": "Connected"
}
$ curl -s -X POST --json '{"url":"http://api2.example.com","state":"Disconnected"}' http://localhost:3000/endpoints | jq .
{
  "id": 1,
  "url": "http://api2.example.com",
  "state": "Disconnected"
}
$ curl -s -X POST --json '{"url":"http://api3.example.com","state":"Connected"}' http://localhost:3000/endpoints | jq .
{
  "id": 2,
  "url": "http://api3.example.com",
  "state": "Connected"
}
```
Now all endpints has an IDs:
  
```shell
$ curl -s -X GET http://localhost:3000/endpoints | jq .
[
  {
    "id": 0,
    "url": "http://api.example.com",
    "state": "Connected"
  },
  {
    "id": 1,
    "url": "http://api2.example.com",
    "state": "Disconnected"
  },
  {
    "id": 2,
    "url": "http://api3.example.com",
    "state": "Connected"
  }
]
```
Getted by ID:

```shell
$ curl -s -X GET http://localhost:3000/endpoints/2 | jq .
{
  "url": "http://api3.example.com",
  "state": "Connected"
}
```
After deleting remaining endpoints keep it's IDs:

```shell
$ curl -s -X DELETE http://localhost:3000/endpoints/1 | jq .
{
  "url": "http://api2.example.com",
  "state": "Disconnected"
}
$ curl -s -X GET http://localhost:3000/endpoints | jq .
[
  {
    "id": 0,
    "url": "http://api.example.com",
    "state": "Connected"
  },
  {
    "id": 2,
    "url": "http://api3.example.com",
    "state": "Connected"
  }
]
```
