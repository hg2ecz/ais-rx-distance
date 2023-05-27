# ais-rx-distance

Hardware requirements:
   Linux (Raspberry or PC) and rtl-sdr dongle.

Usage:
  * build rtl-ais ( https://github.com/hg2ecz/rtl-ais ) and run it (./rtl-ais -n)
  * build this ( cargo build --release )

```
    screen
    1. rtl-ais -n
    2. target/release/ais-rx-distance <receiver_latitude> <receiver_longitude>
```
The result will be printing to the screen and written in the current directory in the dayly log file.
Before a new day (UTC!) will be written a summarize, which 10 MMSI came from the biggest distance.
