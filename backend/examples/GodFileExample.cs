using System;

namespace ExampleProject
{
    // This file has 5 classes (exceeds threshold of 3)
    // Violates single file, single responsibility principle
    
    public class UserService
    {
        public void CreateUser(string name) { }
        public void UpdateUser(int id, string name) { }
        public void DeleteUser(int id) { }
    }

    public class OrderService
    {
        public void CreateOrder(int userId) { }
        public void UpdateOrder(int id) { }
        public void CancelOrder(int id) { }
    }

    public class ProductService
    {
        public void CreateProduct(string name) { }
        public void UpdateProduct(int id, string name) { }
        public void DeleteProduct(int id) { }
    }

    public class ShippingService
    {
        public void CalculateShipping(string address) { }
        public void TrackShipment(string trackingNumber) { }
        public void UpdateShippingAddress(int orderId, string address) { }
    }

    public class PaymentService
    {
        public void ProcessPayment(decimal amount) { }
        public void RefundPayment(int paymentId) { }
        public void ValidatePaymentMethod(string method) { }
    }

    // Additional helper classes (if needed to exceed line count threshold)
    public class EmailNotificationService
    {
        public void SendOrderConfirmation(int orderId) { }
        public void SendShippingUpdate(string trackingNumber) { }
        public void SendPaymentReceipt(int paymentId) { }
    }
}

