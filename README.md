# Zmk battery client

Zmk client connects to specified device and produces json output usable in custom waybar module.
Application retrieves battery levels of all parts of keyboard(both halves in case of split keeb).
The output text is battery percentage of keyboard half with lower value.
The tooltip contains battery levels of all  keyboard parts.

## Usage

Put custom zmk module in waybar:

```

  "custom/zmk": {
    "format": "   {}",
    "tooltip": true,
    "interval": 300,
    "exec": "ZmkBatteryClient E1:E2:69:AB:CE:90",
    "return-type": "json"
  },
```

## Plan

- [ ] By default produce general json output usable for another applications
- [ ] Produce waybar json output only if flag specified(`--waybar`)
