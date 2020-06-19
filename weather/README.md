# Weather

Displays local weather forecast: current temperature and weather conditions.

## Setup

Obtain an OpenWeather API key from [here][1]. Build and install the `weather` block.

```sh
cargo install --path . --root ~/.config/i3blocks/
cp weather.yaml ~/.config/i3blocks/cfg/
```

The `weather` block requires an additional YAML configuration file. Create the
file at `~/.config/i3blocks/cfg/` following the template below.

```yaml
# Configuration YAML for `weather` block
# Required
log_file_path: /absolute/path/to/log/file
open_weather_api_key: OpenWeatherApiKey
temperature_scale: {C, F, K} [default = F]

# Optional
log_geolocation: bool [default = false]
log_ip: bool [default = false]
log_weather_report: bool [default = false]
```

## Usage

Configure i3blocks

```
[weather]
command=~/.config/i3blocks/bin/weather ~/.config/i3blocks/cfg/weather.yaml
interval=1800
```

[1]: https://openweathermap.org/appid
