# Derive from alpine:3.x
FROM alpine:3

# Install rust, cargo, and git
RUN apk add --no-cache gcc musl-dev && apk add --no-cache rust cargo git

# Clone the repository
RUN git clone https://github.com/magnesium-uploader/magnesium-oxide.git /magnesium-oxide

# Build magnesium-oxide for release
RUN cd /magnesium-oxide && cargo build --release

# Create the directory for magnesium-oxide's data
RUN mkdir -p /usr/local/share/magnesium

# Copy the binary to the container
RUN cp /magnesium-oxide/target/release/magnesium-oxide /usr/local/share/magnesium/magnesium-oxide

# Make the binary executable
RUN chmod +x /usr/local/bin/magnesium-oxide

# Clean up
RUN rm -rf /magnesium-oxide
RUN rm -rf /var/cache/apk/*


# Set the working directory the executable will run in
WORKDIR "/usr/local/share/magnesium"

# Run the binary
CMD ["/usr/local/share/magnesium/magnesium-oxide"]