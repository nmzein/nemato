# Nemato

## About

Nemato is a gigapixel image & annotations rendering engine with a web interface for usage in many professional fields for the analysis of extremely large images.

<p align="center">
  <img src="https://github.com/nmzein/nemato/assets/67694622/401968c1-dee3-4080-8634-41fd91aaf4d6" alt="Demo Image" width="700" />
</p>

## 💽 Installation

1. Download the source code: `git clone https://github.com/nmzein/nemato.git`
2. Navigate to main directory: `cd nemato`

---

### Docker (recommended)

1. Install [Docker Engine](https://docs.docker.com/engine/install/).
2. Ensure the Docker daemon is [running](https://docs.docker.com/config/daemon/start/).
3. Run the following commands (note that you may need to run them with `sudo`):

```
# 🛠️ Production [4.81GB; <6 mins on low-perf laptop]
docker-compose up prod

# 🏗️ Development [6.45GB; <3 mins on low-perf laptop]
docker-compose up dev
```

The application can now be accessed at `0.0.0.0:4000`.

#### Useful Docker Commands

Replace any instance of `<branch>` with `prod` or `dev`.

- Use `docker-compose up --build <branch>` to force a rebuild if any files are altered.
- Use `docker exec -it $(docker ps -qf "ancestor=nemato:<branch>") bash` to get a bash shell inside of the container.

---

### Bare Metal

Install dependencies:

| OS     | Command               | Verified Supported Version(s) |
| ------ | --------------------- | ----------------------------- |
| Debian | `./install.sh debian` | 12/Bookworm                   |
| Ubuntu | `./install.sh ubuntu` | -                             |

Build and run:

```
# 🛠️ Production
./run.sh prod

# 🏗️ Development
./run.sh dev
```

The application can now be accessed at `localhost:4000`.
