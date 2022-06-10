# Derive from alpine:3.x
FROM alpine:latest

# Create a user
RUN addgroup -S magnesium && adduser -S magnesium -G magnesium

# Create a directory for magnesium
RUN mkdir /srv/magnesium

# Chown the directory to the user
RUN chown magnesium:magnesium /srv/magnesium

# Copy the binary to the directory
COPY --chown=magnesium:magnesium ./home/runner/work/magnesium-oxide/magnesium-oxide/target/release /srv/magnesium

RUN ls -la /srv/magnesium

# Set the permissions
RUN chmod +x /srv/magnesium/magnesium-oxide

# Set the working directory the executable will run in
WORKDIR /srv/magnesium

# Run the binary
CMD ["/srv/magnesium/magnesium-oxide"]