# desktop2mqtt

A daemon to integrate any pc into your home automation system.
Primarily intended for [Home Assistant](https://home-assistant.io).

## Configuration

Place a `config.yml` file in your working directory with the following contents:

```yaml
mqtt:
  url: <your broker ip/domain>
  username: <your broker username>
  password: <your broker password>
hass:
  entity_id: desktop # will be used to build the different sensors
  name: Max Desktop # will be used for the friendly name of the sensors
modules:
  idle:
    timeout: 5min # duration until this device is reported as unoccupied
    poll_rate: 5s # duration to poll for user input  while the device is unoccupied (optional)
  backlight: none # backlight provider to use (one of: none, stub, raspberry-pi)
  notifications: true # enables notification sending via /desktop2mqtt/entity_id/notify with `{ "title": "", "message": "" }` as payload
  sensors:
    poll_rate: 1s # sensor update rate
    types: # sensors to report
      - type: load
      - type: memory
      - type: core-temperature
      - type: disk-usage
        disks:
          - /
          - /mnt/games
      - type: battery
```
