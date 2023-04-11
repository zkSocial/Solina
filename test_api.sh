#! /bin/sh

# Test the API
curl -X POST -H "Content-Type: application/json" -d '{"names":"John Bob"}' http://localhost:3000/echo