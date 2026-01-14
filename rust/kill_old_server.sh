#!/bin/bash

ps aux | grep latte-album | grep -v grep | awk '{print $2}' | xargs kill -9 && sleep 1 && ss -tlnp | grep 8080