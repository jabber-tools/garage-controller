# Garage Controller

---
[![made-with-Rust](https://img.shields.io/badge/Made%20with-Rust-1f425f.svg)](https://www.rust-lang.org/)
[![Maintenance](https://img.shields.io/badge/Maintained%3F-yes-green.svg)](../../graphs/commit-activity)
[![License](https://img.shields.io/badge/License-Apache-blue.svg)](LICENSE-APACHE)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE-MIT)
[![Build Status](https://travis-ci.org/jabber-tools/garage-controller.svg?branch=master)](https://travis-ci.org/jabber-tools/garage-controller)

Microcontroller software for Raspberry Pi. Based on commands received from MQTT opens/closes garage door utilizing wireless garage door controller connected to GPIO.

[High Level Setup](#high-level-setup)

[GPIO PIN Setup](#gpio-pin-setup)

[Cross-compilation on ARMv6 and ARMv7 architectures](#cross-compilation-on-armv6-and-armv7-architectures)

[Compiling RPPAL library](#compiling-rppal-library)

## High Level Setup
<img src="./examples/docs/img/e2e.png" /></br>

*	User says *open garage door*
*	Virtual assistant translates the voice to text understand user’s intent via NLP.
*	Appropriate fulfillment function (running as HTTP webservice deployed on public cloud provider) is called by virtual assistant. This function will:
       *	Prepare payload for *ToggleGarage* message
       *	Digitally sign it to ensure message integrity. For this JWT (JSON Web tokens, see https://jwt.io/) technology is used
       *	Publishes the message into appropriate queue in public-cloud MQTT provider. 
*	Microcontroller is running MQTT client library and subscribing to MQTT queue. Once it receives the message, appropriate processing will happen:
       * Message is decrypted and verified (both age of the message and digital signature). 
       * Invalid messages are rejected and not processed further.
       * Valid messages are processed. HIGH signal is send for 400 ms into relay input pin. Then pin is set back to LOW value.
*	*Normally open gate* of the relay is closed for 400 ms causing electrical circuit to get closed and electricity to flow in remote garage door controller into soldered pin. This has basically same effect as if user pressed button on remote controller. 
*	Wireless signal is sent to garage door engine and door is open


## GPIO PIN Setup
Pin 7 (GPIO.BOARD layout)/GPIO04 (GPIO.BCM layout) is connected to digital input of relay. NO gate and COM gate are connected to pins of disassembled remote controller of garage door.</br>
<img height="200" src="./examples/docs/img/pin_setup.png" /></br>

## Cross-compilation on ARMv6 and ARMv7 architectures
### Manual cross-compilation setup
See [https://github.com/japaric/rust-cross](https://github.com/japaric/rust-cross)
```
sudo apt-get update
# Install the C cross toolchain
sudo apt-get install -qq gcc-arm-linux-gnueabihf

#Install the cross compiled standard crates
rustup target add armv7-unknown-linux-gnueabihf  #for ARMv7 
rustup target add arm-unknown-linux-gnueabihf #for ARMv6 (Pi Zero)

mkdir -p ~/.cargo
touch ~/.cargo/config

#put following content into file:
[target.armv7-unknown-linux-gnueabihf]
linker = "arm-linux-gnueabihf-gcc"

[target.arm-unknown-linux-gnueabihf]
linker = "arm-linux-gnueabihf-gcc"

#ARMv7 build
cargo build --target=armv7-unknown-linux-gnueabihf

#ARMv6 build
cargo build --target=arm-unknown-linux-gnueabihf
```
Nice details on compilation for Raspberry Pi Zero (ARMv6) can be found [here](https://disconnected.systems/blog/rust-powered-rover/#setting-up-rust-for-cross-compiling).

### Compiling RPPAL library
In order to compile following dependency [RPPAL](https://github.com/golemparts/rppal) CC compiler must be installed otherwise following error will be thrown:
error: linker `cc` not found
could not compile `libc`.

Details [here](https://ostechnix.com/how-to-fix-rust-error-linker-cc-not-found-on-linux/)

Solution is to run following command:
```
sudo apt install build-essential
```
Nice article on rust cross-compilation to arm architecture can be also found here [here](https://www.growse.com/2020/04/26/adventures-in-rust-and-cross-compilation-for-the-raspberry-pi.html).


### Out-of-the-box cross-compilation setup
While manual procedure described above works fine for AMR7, AMR6 was not working. Cross-compilation using target *arm-unknown-linux-gnueabihf* finished without error but executable was crashing on Raspberry Pi Zero W with error *Illegal Instruction*. This almost seems as if it was in fact compiled into ARM7 which has additional instructions (i.e. non-compatible with ARM6).
Because of this it is better to use out of the box docker images with all compilation targets preconfigured properly:) (together with rustc, git, ssh etc.).
Details described [here](https://piers.rocks/docker/containers/raspberry/pi/rust/cross/compile/compilation/2018/12/16/rust-compilation-for-raspberry-pi.html), respective docker container described [here](https://hub.docker.com/r/piersfinlayson/build). Basically:

```
docker pull piersfinlayson/build

docker run --rm -ti -v ~/builds:/home/build/builds piersfinlayson/build

build@f435ef72a98f:~$ pwd
/home/build
build@f435ef72a98f:~$ cat ~/.cargo/config
[target.x86_64-unknown-linux-gnu]
linker = "x86_64-linux-gnu-gcc"

[target.x86_64-unknown-linux-musl]
linker = "x86_64-linux-musl-gcc"

[target.armv7-unknown-linux-gnueabihf]
linker = "arm-linux-gnueabihf-gcc"

[target.armv7-unknown-linux-musleabihf]
linker = "armv7-linux-musleabihf-gcc"

[target.arm-unknown-linux-gnueabihf]
linker = "armv6-linux-gnueabihf-gcc"

[target.arm-unknown-linux-musleabihf]
linker = "armv6-linux-musleabihf-gcc"


# compile to ARM6 or ARM7
cargo build --target=arm-unknown-linux-gnueabihf
cargo build --target=armv7-unknown-linux-gnueabihf

# copy to respective Raspberry Pi using scp command
scp garage-controller pi@192.168.1.104:/tmp

```
