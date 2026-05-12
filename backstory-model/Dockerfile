FROM python:3.14.4-slim-trixie

LABEL maintainer="rychkin06@gmail.com"

ENV PYTHONUNBUFFERED=1
ENV DEBIAN_FRONTEND=noninteractive

WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        build-essential \
        git \
        curl \
        ca-certificates \
        libnuma-dev \
    && rm -rf /var/lib/apt/lists/*

COPY requirements.txt ./

RUN pip install --no-cache-dir -r requirements.txt \
    && pip install --no-cache-dir gunicorn

COPY . /app

EXPOSE 5000

HEALTHCHECK --interval=5m --timeout=10s --retries=3 \
  CMD curl -f http://localhost:5000/health || exit 1

CMD ["gunicorn", "-w", "1", "-k", "gthread", "--threads", "2", "-b", "0.0.0.0:5000", "app:app"]
