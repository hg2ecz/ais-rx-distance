# ais-rx-distance

Hardware requirements:
   Linux (Raspberry or PC) and rtl-sdr dongle.

Usage:
  * build rtl-ais: https://github.com/hg2ecz/rtl-ais
  * build this ( cargo build --release )

```
   Then run:
    1. rtl-ais -n                                                              # receive raw AIS with RTL-SDR hardware
    2. target/release/ais-rx-distance <receiver_latitude> <receiver_longitude> # decode and calculate distances
```
The result will be printed to the screen and written to a daily log file.
Before a new day (UTC), a summary will be written, including the top 10 MMSIs with the greatest distances.
