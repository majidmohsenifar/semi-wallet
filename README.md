# Semi-Wallet
Have you ever done installing a crypto wallet then insert your seed phrase only for knowing how much balance you have? 
This is a simple project designed to track addresses you are interested in, you can have all your own addresses in one place and check how much you have gained or lose without worrying about
having your seed phrases always with you. in this project you can register and buy a plan to track your crypto addresses balance and the equivalent USD amount of them.


## How to run
1. Clone the repository
2. run docker-compose up -d
3. create .env and copy the sample.env content into it
4. run project using ```cargo run --bin server```
5. open the swagger in your browser ```http://127.0.0.1:8000/swagger-ui```

## Features
- [x] register user
- [x] login user 
- [x] authenticate using jwt
- [x] buy plan 
- [x] pay for plan using stripe
- [x] insert user-coin
- [x] get user-coins list including the usd equivalent amount of them
- [x] delete user-coin 
- [x] update user-coin

## Binaries
- server: The http server serving rest api ```cargo run --bin server```
- cli: Cron jobs to update user coins amount  ```cargo run --bin cli```
- ws: get the coin prices from binance  ```cargo run --bin ws```

## Test
project contains only integration test placed in test module
- run ```cargo test``` to run the tests

## TODO:
- add coins to db using migration
- check the validity of address in create-user-coin
- handle the case when the user tries to add the same coin address twice
- add http-server container to docker-compose
- add cli container to docker-compose
- add ws container to docker-compose
- add sqlx migration container to docker-compose
- remove unwrap calls
- remove clone calls where ever it is possible
