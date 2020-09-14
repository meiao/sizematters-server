# SizeMatters

SizeMatters is a ticket sizing (poker planning) webapp to make those long grooming sessions a little more
tolerable.

One of the goals is to create a tool that does not track users and has a minimal entry barrier.
For that it does not require user creation, and forgets everything about the users as soon as they 
leave.

You can check it at https://sizematters.dev

## sizematters-server

This repository is the server component of SizeMatters.
It is written in Rust, using [Actix](https://actix.rs).

## sizematters-ui

The UI component of the project uses Vue to create an SPA.
Check it out at <https://github.com/meiao/sizematters-ui/>

# Design

Actix was selected because it combines two technologies that I wanted to use in this project:
- Websockets - for real-time data reception on clients;
- an actor framework - for fast and easy concurrency.

The actor code is heavily influenced by my current experience with Akka and Java in general.

