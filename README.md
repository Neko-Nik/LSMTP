# **LSMTPD - Lightweight SMTP Daemon**

A minimalistic, lightweight SMTP server for receiving mail and forwarding to AMQP. It's lightweight, high-performance, and easy to configure for your needs.

## **Installation & Setup**  

### **1. Prerequisites**

Ensure you have the following installed:

- **Rust** (latest stable version) → [Install Rust](https://www.rust-lang.org/tools/install)  
- **Cargo** (comes with Rust)  
- **RabbitMQ** (for message queuing) or any other AMQP server  

### **2. Environment Variables**  

Before running LSMTPD, set the required environment variables:  

```sh
export RUST_LOG=lsmtpd=TRACE
export BIND_ADDRESS=0.0.0.0
export BIND_PORT=25
export SERVER_NAME=nekonik.com
export AMQP_HOST=rabbitmq.nekonik.com
export AMQP_PORT=5672
export AMQP_USERNAME=admin
export AMQP_PASSWORD=admin
export AMQP_VHOST=/
export AMQP_EXCHANGE=lsmtp
export AMQP_ROUTING_KEY=lsmtp
```

Make sure to replace any missing values, like `AMQP_PORT`, with the correct configuration.  

### **3. Running the Server**  

Once the environment variables are set, start the SMTP daemon using:  

```sh
cargo run -r
```

This compiles and runs the project in **release mode** for better performance.  

---

## **Configuration**

- **Logging:** Uses Rust's `RUST_LOG` for debugging (`TRACE` mode)
- **Binding:** Listens on `0.0.0.0:25` by default
- **AMQP Integration:** Messages are forwarded using the provided any AMQP server tested for RabbitMQ

---

## **Development & Contribution**  

To contribute:

1. Fork the repository
2. Clone or fork the [repository](https://github.com/Neko-Nik/LSMTP)
3. Install dependencies and to run the project:
   ```sh
   cargo run -r
   ```
4. Make changes, test, and submit a pull request.

---

### **Need Help?**  

For issues, open a ticket in the [GitHub Issues](https://github.com/Neko-Nik/LSMTP/issues) 🚀  

---

## **License**  

This project is licensed under [GPL-3.0](https://github.com/Neko-Nik/LSMTP/blob/main/LICENSE)
