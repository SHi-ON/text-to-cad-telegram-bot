# Text-to-CAD Telegram Bot

Text-to-CAD Telegram bot based on
Zoo's [Text-to-CAD](https://zoo.dev/text-to-cad) API.
Get a 3D model for your given text prompt right in Telegram!

Try it on Telegram:
https://t.me/text_to_cad_unofficial_bot

## Authentication

You need the following variables loaded in your environment to run your own bot:

* `KITTYCAD_API_TOKEN`: The access token (obtained by logging into your Zoo
  account).
* `TELOXIDE_TOKEN`: Your telegram bot token (obtained from BotFather).

### TODO

* [ ] Choose model file format
* [ ] State management and persistence (e.g. file format preferences)
* [ ] More testing on the cache hit side
* [x] Pending animation
* [ ] Dockerize
* [ ] Model file name handling (e.g. ellipsize-ing, etc.)

### Contribution

All contributions of any kind are welcome:)


### Disclaimer

This repository is not affiliated with Zoo.