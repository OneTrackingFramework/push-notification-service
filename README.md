# push-service
Microservice written in Rust to send push notifications to clients (e.g. iOS / Android).
- Apache Kafka consumer
- Stateless desgin
- Lightweight
- Platform independant
- Docker
  
This microservice is implemented for the One Tracking Framework, which was originally developed by the #wirvsvirus-Hackaton. The goal is to curb the Covid-19 disease.

## Test with Docker environment
```
docker-compose up -d  # setup environment
RUST_LOG=info cargo run
```
Hint: dont forget to create kafka topics

## Create Docker image
```
docker build -t pushy-image .
docker run pushy-image pushy
```
