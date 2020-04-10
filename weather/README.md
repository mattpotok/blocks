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
check_connection: bool  # Check if there is an internet connection
log_extra: bool  # Log extra information (IP, geolocation, weather)
log_file_path: /absolute/path/to/log/file
open_weather_api_key: OpenWeatherApiKey
temperature_units: [C, F, K]  # Desired temperature units
```

## Usage

Configure i3blocks

```
[weather]
command=~/.config/i3blocks/bin/battery ~/.config/i3blocks/cfg/battery.yaml
interval=1800
```

[1]: https://openweathermap.org/appid
