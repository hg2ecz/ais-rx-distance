# ais-rx-distance

Hardware requirements:
   Linux (Raspberry or PC) and rtl-sdr dongle.

Usage:
  * build rtl-ais: https://github.com/hg2ecz/rtl-ais
  * build this ( cargo build --release )

```
    It can running in two terminals (multiple terminal or screen or nohup)
    1. rtl-ais -n                                                              # receive raw AIS
    2. target/release/ais-rx-distance <receiver_latitude> <receiver_longitude> # decode and calculate distance
```
The result will be printing to the screen and writing to daily log file.
Before a new day (UTC!) will be written a summarize, which 10 MMSI came from the biggest distance.
