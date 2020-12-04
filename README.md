# Evdev Proxy

Proxy multiple evdev input devices to single virtual uinput device.

Primary usecase are - passthrough input events to qemu-evdev device with 
support of input device hotplug/unplug. For example using some kvm-switches
will detach physical evdev devices from qemu on first switch to another 
machine and only way to attach it back is to restart the guest.

Note:

This is early version, WIP.

## Installation

## Configuration

Configuration is pretty straightforward, please refer to example `config.toml`.

### How to use with QEmu

Assuming you use example devices from `config.toml`, just add following args 
to your qemu command line:

    -device virtio-keyboard-pci,id=input2,bus=pci.10,addr=0x0 \
    -device virtio-mouse-pci,id=input3,bus=pci.11,addr=0x0 \
    -object input-linux,id=aio,evdev=/dev/input/by-id/virtual-event-EvdevProxyAIO,grab_all=on,repeat=on

Or this if you want to use separate keyboard and mouse devices:

    -device virtio-keyboard-pci,id=input2,bus=pci.10,addr=0x0 \
    -device virtio-mouse-pci,id=input3,bus=pci.11,addr=0x0 \
    -object input-linux,id=kbd,evdev=/dev/input/by-id/virtual-event-EvdevProxyKeyboard,grab_all=on,repeat=on \
    -object input-linux,id=mouse,evdev=/dev/input/by-id/virtual-event-EvdevProxyMouse

If you use `libvirt` you can do the same with following domain XML:

    <domain xmlns:qemu="http://libvirt.org/schemas/domain/qemu/1.0" type="kvm">
      ...
      <input type="keyboard" bus="virtio">
        <address type="pci" domain="0x0000" bus="0x0a" slot="0x00" function="0x0"/>
      </input>
      <input type="mouse" bus="virtio">
        <address type="pci" domain="0x0000" bus="0x0b" slot="0x00" function="0x0"/>
      </input>
      ...
      <qemu:commandline>
        <qemu:arg value="-object"/>
        <qemu:arg value="input-linux,id=aio,evdev=/dev/input/by-id/virtual-event-EvdevProxyAIO,grab_all=yes,repeat=yes"/>
      </qemu:commandline>
    </domain>

Of this in case of separate devices:

    <domain xmlns:qemu="http://libvirt.org/schemas/domain/qemu/1.0" type="kvm">
      ...
      <input type="keyboard" bus="virtio">
        <address type="pci" domain="0x0000" bus="0x0a" slot="0x00" function="0x0"/>
      </input>
      <input type="mouse" bus="virtio">
        <address type="pci" domain="0x0000" bus="0x0b" slot="0x00" function="0x0"/>
      </input>
      ...
      <qemu:commandline>
        <qemu:arg value="-object"/>
        <qemu:arg value="input-linux,id=kbd,evdev=/dev/input/by-id/virtual-event-EvdevProxyKeyboard,grab_all=yes,repeat=yes"/>
        <qemu:arg value="-object"/>
        <qemu:arg value="input-linux,id=mouse,evdev=/dev/input/by-id/virtual-event-EvdevProxyMouse"/>
      </qemu:commandline>
    </domain>

**NOTE**: Do not forget to add proper `xmlns:qemu` attribute to domain's root `<domain>` tag (see examples above).

On starting of qemu domain it should grab specified evdev devices. To switch between host and guest press both 
`Right Ctrl` and `Left Ctrl` keys (this is default qemu behavior).
You can change default grab combination by adding `grab-toggle` parameter to `input-linux` object definition.

Currently supported values are:
  * ctrl-ctrl
  * alt-alt
  * meta-meta
  * scrolllock
  * ctrl-scrolllock

For example:

    -object input-linux,id=aio,evdev=/dev/input/by-id/virtual-event-EvdevProxyAIO,grab_all=on,repeat=on,grab-toggle=alt-alt

For more information see following links:
  * https://github.com/qemu/qemu/commit/2657846fb2e47e8ba847b5ef6fe742466414c745
  * https://github.com/qemu/qemu/blob/master/ui/input-linux.c

## Troubleshooting
### Domain refuses to start
If for some reason domain refuses to start, complaining about missing evdev device, try following:
 * Ensure that /dev/uinput is accessible to the evdev-proxy.
 * Make sure that evdev-proxy is started, and check its logs for any errors regarding device creation.
 * If numerical evdev device (e.g `/dev/input/eventN`) is created, but no symlink is created for it in `/dev/input/by-id/`
   make sure that you your virtual device name has `EvdevProxy` prefix in its name or create/edit corresponding udev rule
   in `70-uinput-evdev-proxy.rules` file, or create your own.

### Domain starts, but no keyboard/mouse in guest
If domains starts fine, but no events passed to guest domain try following:
 * Double-check that valid `input-linux` objects are passed to qemu command line (`ps aux | grep qemu`)
 * Check evdev-proxy logs for messages about source device captures (e.g `Added new source dev "/dev/input/eventN"`),
   also double-check selector parameters if `config.toml`.
   
### Mouse/Keyboard goes crazy in guest
If you experience weird behaviour when trying to move the mouse and pressing keyboard keys simultaneously like 
spurious mouse movements and key presses try following:
 * Make sure that you added `virtio-input` input devices to domain configuration (see above)
 * Make sure that guest OS use those `virtio-input` devices instead of PS/2 emulation. 
   
 * On linux you can check which devices xinput use, first get xinput IDs for those devices:
   
       xinput | grep -i virtio

   Then test that xinput events are coming from them:

       xinput test-xi2

 * On Windows make sure that `virtio-input` drivers are installed and devices are started properly in Device Manager.
   For latest/stable signed virtio drivers check [fedora linux site](https://docs.fedoraproject.org/en-US/quick-docs/creating-windows-virtual-machines-using-virtio-drivers/#virtio-win-direct-downloads).