# push-service
Microservice written in Rust to send push notifications to clients (e.g. iOS / Android).
- Apache Kafka consumer
- Stateless desgin
- Lightweight
- Platform independant
- Docker
  
This microservice is implemented for the One Tracking Framework, which was originally developed by the #wirvsvirus-Hackaton. The goal is to curb the Covid-19 disease.

## Run with Docker

```
docker-compose up -d
docker build -t pushy-image .
```
Hint: dont forget to create to create db schema and kafka topics
