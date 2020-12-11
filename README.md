# desktop2mqtt

A daemon to integrate any pc into your home automation system.
Primarily intended for [Home Assistant](https://home-assistant.io).

## Configuration

Place a `config.yml` file in your working directory with the following contents:

```yaml
mqtt:
  url: <your broker ip/domain>
hass:
  entity_id: desktop # will be used to build the different sensors
  name: Max Desktop # will be used for the friendly name of the sensors
idle_timeout: 300 # time in seconds until this device is reported as unoccupied
idle_poll_rate: 5 # time in seconds to poll for user input while the device is unoccupied (optional)
backlight: none # backlight provider to use (one of: none, stub, raspberry-pi)
```
