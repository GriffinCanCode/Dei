// God class example - does way too much

using System;
using System.Collections.Generic;
using System.Linq;
using System.IO;
using System.Text;

namespace TestApp.BadDesign
{
    /// <summary>
    /// Massive order manager that handles everything - BAD DESIGN
    /// </summary>
    public class MegaOrderManager
    {
        private readonly Dictionary<int, Order> _orders;
        private readonly Dictionary<int, Customer> _customers;
        private readonly Dictionary<string, Product> _products;
        private readonly Dictionary<int, Shipment> _shipments;
        private readonly Dictionary<int, Invoice> _invoices;
        private readonly List<string> _auditLog;
        private readonly Dictionary<int, PaymentInfo> _payments;
        private readonly Dictionary<string, decimal> _discounts;
        private readonly List<EmailMessage> _emailQueue;
        private readonly Dictionary<int, List<Notification>> _notifications;

        public MegaOrderManager()
        {
            _orders = new Dictionary<int, Order>();
            _customers = new Dictionary<int, Customer>();
            _products = new Dictionary<string, Product>();
            _shipments = new Dictionary<int, Shipment>();
            _invoices = new Dictionary<int, Invoice>();
            _auditLog = new List<string>();
            _payments = new Dictionary<int, PaymentInfo>();
            _discounts = new Dictionary<string, decimal>();
            _emailQueue = new List<EmailMessage>();
            _notifications = new Dictionary<int, List<Notification>>();
        }

        // Order management
        public int CreateOrder(int customerId, List<OrderItem> items, string shippingAddress)
        {
            if (!_customers.ContainsKey(customerId))
                throw new ArgumentException("Customer not found");

            var orderId = _orders.Count + 1;
            var order = new Order
            {
                Id = orderId,
                CustomerId = customerId,
                Items = items,
                ShippingAddress = shippingAddress,
                Status = "pending",
                CreatedAt = DateTime.Now
            };

            _orders[orderId] = order;
            LogAudit($"Order {orderId} created for customer {customerId}");
            SendOrderConfirmationEmail(customerId, orderId);
            UpdateInventory(items);
            
            return orderId;
        }

        public void UpdateOrderStatus(int orderId, string status)
        {
            if (!_orders.ContainsKey(orderId))
                throw new ArgumentException("Order not found");

            _orders[orderId].Status = status;
            LogAudit($"Order {orderId} status changed to {status}");
            NotifyCustomer(orderId, $"Your order status is now {status}");
        }

        public void CancelOrder(int orderId)
        {
            if (!_orders.ContainsKey(orderId))
                throw new ArgumentException("Order not found");

            var order = _orders[orderId];
            order.Status = "cancelled";
            RefundPayment(orderId);
            RestoreInventory(order.Items);
            LogAudit($"Order {orderId} cancelled");
            SendCancellationEmail(order.CustomerId, orderId);
        }

        public Order GetOrder(int orderId)
        {
            return _orders.ContainsKey(orderId) ? _orders[orderId] : null;
        }

        public List<Order> GetOrdersByCustomer(int customerId)
        {
            return _orders.Values.Where(o => o.CustomerId == customerId).ToList();
        }

        public List<Order> GetOrdersByStatus(string status)
        {
            return _orders.Values.Where(o => o.Status == status).ToList();
        }

        // Customer management
        public int CreateCustomer(string name, string email, string phone, string address)
        {
            ValidateEmail(email);
            ValidatePhone(phone);

            var customerId = _customers.Count + 1;
            var customer = new Customer
            {
                Id = customerId,
                Name = name,
                Email = email,
                Phone = phone,
                Address = address,
                CreatedAt = DateTime.Now
            };

            _customers[customerId] = customer;
            LogAudit($"Customer {customerId} created: {name}");
            SendWelcomeEmail(email);
            
            return customerId;
        }

        public void UpdateCustomer(int customerId, string name, string email, string phone, string address)
        {
            if (!_customers.ContainsKey(customerId))
                throw new ArgumentException("Customer not found");

            var customer = _customers[customerId];
            customer.Name = name ?? customer.Name;
            customer.Email = email ?? customer.Email;
            customer.Phone = phone ?? customer.Phone;
            customer.Address = address ?? customer.Address;

            LogAudit($"Customer {customerId} updated");
        }

        public void DeleteCustomer(int customerId)
        {
            if (!_customers.ContainsKey(customerId))
                throw new ArgumentException("Customer not found");

            _customers.Remove(customerId);
            _orders.Where(kvp => kvp.Value.CustomerId == customerId)
                   .ToList()
                   .ForEach(kvp => _orders.Remove(kvp.Key));
            
            LogAudit($"Customer {customerId} deleted");
        }

        public Customer GetCustomer(int customerId)
        {
            return _customers.ContainsKey(customerId) ? _customers[customerId] : null;
        }

        // Product management
        public void AddProduct(string sku, string name, decimal price, int stock)
        {
            ValidatePrice(price);

            var product = new Product
            {
                Sku = sku,
                Name = name,
                Price = price,
                Stock = stock
            };

            _products[sku] = product;
            LogAudit($"Product {sku} added: {name}");
        }

        public void UpdateProduct(string sku, string name, decimal price, int stock)
        {
            if (!_products.ContainsKey(sku))
                throw new ArgumentException("Product not found");

            var product = _products[sku];
            product.Name = name ?? product.Name;
            product.Price = price > 0 ? price : product.Price;
            product.Stock = stock >= 0 ? stock : product.Stock;

            LogAudit($"Product {sku} updated");
        }

        public void DeleteProduct(string sku)
        {
            _products.Remove(sku);
            LogAudit($"Product {sku} deleted");
        }

        public Product GetProduct(string sku)
        {
            return _products.ContainsKey(sku) ? _products[sku] : null;
        }

        // Payment processing
        public void ProcessPayment(int orderId, string paymentMethod, string transactionId)
        {
            if (!_orders.ContainsKey(orderId))
                throw new ArgumentException("Order not found");

            var order = _orders[orderId];
            var amount = CalculateOrderTotal(order);

            var payment = new PaymentInfo
            {
                OrderId = orderId,
                Amount = amount,
                Method = paymentMethod,
                TransactionId = transactionId,
                ProcessedAt = DateTime.Now,
                Status = "completed"
            };

            _payments[orderId] = payment;
            UpdateOrderStatus(orderId, "paid");
            GenerateInvoice(orderId);
            LogAudit($"Payment processed for order {orderId}: ${amount}");
        }

        public void RefundPayment(int orderId)
        {
            if (!_payments.ContainsKey(orderId))
                throw new ArgumentException("No payment found for order");

            var payment = _payments[orderId];
            payment.Status = "refunded";
            LogAudit($"Payment refunded for order {orderId}: ${payment.Amount}");
        }

        // Shipping management
        public void CreateShipment(int orderId, string trackingNumber, string carrier)
        {
            if (!_orders.ContainsKey(orderId))
                throw new ArgumentException("Order not found");

            var shipmentId = _shipments.Count + 1;
            var shipment = new Shipment
            {
                Id = shipmentId,
                OrderId = orderId,
                TrackingNumber = trackingNumber,
                Carrier = carrier,
                Status = "in_transit",
                ShippedAt = DateTime.Now
            };

            _shipments[shipmentId] = shipment;
            UpdateOrderStatus(orderId, "shipped");
            SendShippingNotification(orderId, trackingNumber);
            LogAudit($"Shipment {shipmentId} created for order {orderId}");
        }

        public void UpdateShipmentStatus(int shipmentId, string status)
        {
            if (!_shipments.ContainsKey(shipmentId))
                throw new ArgumentException("Shipment not found");

            _shipments[shipmentId].Status = status;
            LogAudit($"Shipment {shipmentId} status updated to {status}");
        }

        // Invoice management
        public void GenerateInvoice(int orderId)
        {
            if (!_orders.ContainsKey(orderId))
                throw new ArgumentException("Order not found");

            var order = _orders[orderId];
            var invoiceId = _invoices.Count + 1;
            var invoice = new Invoice
            {
                Id = invoiceId,
                OrderId = orderId,
                Amount = CalculateOrderTotal(order),
                GeneratedAt = DateTime.Now
            };

            _invoices[invoiceId] = invoice;
            SendInvoiceEmail(order.CustomerId, invoiceId);
            LogAudit($"Invoice {invoiceId} generated for order {orderId}");
        }

        // Email notifications
        private void SendOrderConfirmationEmail(int customerId, int orderId)
        {
            var customer = _customers[customerId];
            var email = new EmailMessage
            {
                To = customer.Email,
                Subject = $"Order {orderId} Confirmation",
                Body = $"Your order {orderId} has been confirmed"
            };
            _emailQueue.Add(email);
        }

        private void SendCancellationEmail(int customerId, int orderId)
        {
            var customer = _customers[customerId];
            var email = new EmailMessage
            {
                To = customer.Email,
                Subject = $"Order {orderId} Cancelled",
                Body = $"Your order {orderId} has been cancelled"
            };
            _emailQueue.Add(email);
        }

        private void SendWelcomeEmail(string email)
        {
            var msg = new EmailMessage
            {
                To = email,
                Subject = "Welcome!",
                Body = "Welcome to our store"
            };
            _emailQueue.Add(msg);
        }

        private void SendShippingNotification(int orderId, string trackingNumber)
        {
            var order = _orders[orderId];
            var customer = _customers[order.CustomerId];
            var email = new EmailMessage
            {
                To = customer.Email,
                Subject = $"Order {orderId} Shipped",
                Body = $"Tracking: {trackingNumber}"
            };
            _emailQueue.Add(email);
        }

        private void SendInvoiceEmail(int customerId, int invoiceId)
        {
            var customer = _customers[customerId];
            var email = new EmailMessage
            {
                To = customer.Email,
                Subject = $"Invoice {invoiceId}",
                Body = "Please find your invoice attached"
            };
            _emailQueue.Add(email);
        }

        // Notification management
        private void NotifyCustomer(int orderId, string message)
        {
            var order = _orders[orderId];
            var notification = new Notification
            {
                Message = message,
                CreatedAt = DateTime.Now
            };

            if (!_notifications.ContainsKey(order.CustomerId))
                _notifications[order.CustomerId] = new List<Notification>();

            _notifications[order.CustomerId].Add(notification);
        }

        // Inventory management
        private void UpdateInventory(List<OrderItem> items)
        {
            foreach (var item in items)
            {
                if (_products.ContainsKey(item.Sku))
                {
                    _products[item.Sku].Stock -= item.Quantity;
                    LogAudit($"Inventory updated for {item.Sku}: -{item.Quantity}");
                }
            }
        }

        private void RestoreInventory(List<OrderItem> items)
        {
            foreach (var item in items)
            {
                if (_products.ContainsKey(item.Sku))
                {
                    _products[item.Sku].Stock += item.Quantity;
                    LogAudit($"Inventory restored for {item.Sku}: +{item.Quantity}");
                }
            }
        }

        // Discount management
        public void ApplyDiscount(int orderId, string discountCode)
        {
            if (!_orders.ContainsKey(orderId))
                throw new ArgumentException("Order not found");

            if (!_discounts.ContainsKey(discountCode))
                throw new ArgumentException("Invalid discount code");

            LogAudit($"Discount {discountCode} applied to order {orderId}");
        }

        public void AddDiscountCode(string code, decimal percentage)
        {
            _discounts[code] = percentage;
            LogAudit($"Discount code {code} added: {percentage}%");
        }

        // Calculation helpers
        private decimal CalculateOrderTotal(Order order)
        {
            decimal total = 0;
            foreach (var item in order.Items)
            {
                if (_products.ContainsKey(item.Sku))
                {
                    total += _products[item.Sku].Price * item.Quantity;
                }
            }
            return total;
        }

        // Validation helpers
        private void ValidateEmail(string email)
        {
            if (string.IsNullOrEmpty(email) || !email.Contains("@"))
                throw new ArgumentException("Invalid email");
        }

        private void ValidatePhone(string phone)
        {
            if (string.IsNullOrEmpty(phone) || phone.Length < 10)
                throw new ArgumentException("Invalid phone number");
        }

        private void ValidatePrice(decimal price)
        {
            if (price <= 0)
                throw new ArgumentException("Price must be positive");
        }

        // Audit logging
        private void LogAudit(string message)
        {
            _auditLog.Add($"[{DateTime.Now}] {message}");
        }

        public List<string> GetAuditLog()
        {
            return new List<string>(_auditLog);
        }

        public void ExportAuditLog(string filePath)
        {
            File.WriteAllLines(filePath, _auditLog);
        }
    }

    // Supporting classes
    public class Order
    {
        public int Id { get; set; }
        public int CustomerId { get; set; }
        public List<OrderItem> Items { get; set; }
        public string ShippingAddress { get; set; }
        public string Status { get; set; }
        public DateTime CreatedAt { get; set; }
    }

    public class OrderItem
    {
        public string Sku { get; set; }
        public int Quantity { get; set; }
    }

    public class Customer
    {
        public int Id { get; set; }
        public string Name { get; set; }
        public string Email { get; set; }
        public string Phone { get; set; }
        public string Address { get; set; }
        public DateTime CreatedAt { get; set; }
    }

    public class Product
    {
        public string Sku { get; set; }
        public string Name { get; set; }
        public decimal Price { get; set; }
        public int Stock { get; set; }
    }

    public class PaymentInfo
    {
        public int OrderId { get; set; }
        public decimal Amount { get; set; }
        public string Method { get; set; }
        public string TransactionId { get; set; }
        public DateTime ProcessedAt { get; set; }
        public string Status { get; set; }
    }

    public class Shipment
    {
        public int Id { get; set; }
        public int OrderId { get; set; }
        public string TrackingNumber { get; set; }
        public string Carrier { get; set; }
        public string Status { get; set; }
        public DateTime ShippedAt { get; set; }
    }

    public class Invoice
    {
        public int Id { get; set; }
        public int OrderId { get; set; }
        public decimal Amount { get; set; }
        public DateTime GeneratedAt { get; set; }
    }

    public class EmailMessage
    {
        public string To { get; set; }
        public string Subject { get; set; }
        public string Body { get; set; }
    }

    public class Notification
    {
        public string Message { get; set; }
        public DateTime CreatedAt { get; set; }
    }
}

