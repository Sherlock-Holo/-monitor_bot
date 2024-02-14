# monitor_bot

a monitor telegram bot

## support

- memory monitor
- memory max usage ratio notify
- disable/enable notify

## usage

1. create a telegram bot
2. register `/show` command
3. register `/setting` command
4. start **monitor_bot**

## help

```shell
Usage: monitor_bot [OPTIONS] --bot-token <BOT_TOKEN> --group-chat-id <GROUP_CHAT_ID>

Options:
  -d, --debug                                      enable debug log
      --mem-watch-interval <MEM_WATCH_INTERVAL>    set memory watch interval [default: 3s]
      --mem-max-usage-ratio <MEM_MAX_USAGE_RATIO>  set memory max usage ratio [default: 0.7]
  -b, --bot-token <BOT_TOKEN>                      telegram bot token
      --group-chat-id <GROUP_CHAT_ID>              telegram group chat id
  -h, --help                                       Print help
```
