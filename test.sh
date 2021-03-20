#!/bin/bash

URL="http://localhost:8000"

# curl $URL/register -X POST -H "Content-Type: application/json" -d '{"username": "aggelalex", "email": "ubuntu1aggelalex@gmail.com", "password": "yomama"}'
curl $URL/login -X POST -H "Content-Type: application/json" -d '{"username": "sjk", "password": "jljkl"}'