# Derive from debian image
FROM debian:bullseye-slim

RUN apt update && apt install -y \
    apt-transport-https \
    ca-certificates \
    curl \
    gnupg-agent \
    software-properties-common \
    build-essential

# Create a user for the container called "magnesium"
RUN useradd -m -s /bin/bash magnesium

# Create a directory to hold oxide
RUN mkdir /srv/magnesium

# Chown the directory to the magnesium user
RUN chown magnesium:magnesium /srv/magnesium

# Set the current working directory to the oxide directory
WORKDIR /srv/magnesium

# Change into the magnesium user
RUN su - magnesium

# Copy the executable file to the container
COPY --chown=magnesium:magnesium ./target/release/magnesium-oxide .

# Make the file executable
RUN chmod +x magnesium-oxide

# Expose the port 8080
EXPOSE 8080 

# Run the executable file
CMD ["./magnesium-oxide"]