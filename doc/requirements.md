# Requirements for the One Tracking Framework Push Service

## Functional 

* Send push notifications to different kind of clients
    * iOS
    * Firebase (Android)
* Manage user to device / app mapping in stateless manner
* Provide consumer for event store
    * Create user to device / app mapping
    * Delete user to device / app mapping
    * Send push notification to user

## Non Functional

* The data exchange at interfaces is encrypted according to the state of the art
* The microservice shall be able to handle >> 1Mio Users 
* The microservice shall be able to send > 1000 notifications per second 