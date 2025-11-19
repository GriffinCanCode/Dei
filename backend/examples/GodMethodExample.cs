using System;
using System.Collections.Generic;
using System.Linq;

namespace ExampleProject
{
    /// <summary>
    /// Example of a class with god methods (overly complex methods)
    /// </summary>
    public class OrderProcessor
    {
        // This method has too many parameters (6 > threshold of 5)
        public Order ProcessOrder(
            string customerId,
            List<Product> products,
            string shippingAddress,
            string billingAddress,
            PaymentMethod paymentMethod,
            string couponCode)
        {
            // ... method body ...
            var order = new Order
            {
                CustomerId = customerId,
                Products = products,
                ShippingAddress = shippingAddress,
                BillingAddress = billingAddress,
                PaymentMethod = paymentMethod,
                CouponCode = couponCode
            };
            return order;
        }

        // This method is too long (>50 lines) and too complex (complexity > 10)
        public decimal CalculateOrderTotal(Order order)
        {
            decimal total = 0;
            decimal tax = 0;
            decimal shipping = 0;
            decimal discount = 0;

            // Calculate item totals
            foreach (var item in order.Products)
            {
                if (item.IsOnSale)
                {
                    if (item.SalePercentage > 50)
                    {
                        total += item.Price * 0.5m;
                    }
                    else if (item.SalePercentage > 25)
                    {
                        total += item.Price * 0.75m;
                    }
                    else
                    {
                        total += item.Price * (1 - item.SalePercentage / 100);
                    }
                }
                else
                {
                    total += item.Price;
                }
            }

            // Apply coupon
            if (!string.IsNullOrEmpty(order.CouponCode))
            {
                if (order.CouponCode == "SAVE10")
                {
                    discount = total * 0.1m;
                }
                else if (order.CouponCode == "SAVE20")
                {
                    discount = total * 0.2m;
                }
                else if (order.CouponCode == "SAVE30")
                {
                    discount = total * 0.3m;
                }
                else if (order.CouponCode == "FREESHIP")
                {
                    shipping = 0;
                }
            }

            // Calculate tax based on region
            if (order.ShippingAddress.Contains("CA"))
            {
                tax = total * 0.0975m;
            }
            else if (order.ShippingAddress.Contains("NY"))
            {
                tax = total * 0.08875m;
            }
            else if (order.ShippingAddress.Contains("TX"))
            {
                tax = total * 0.0825m;
            }
            else
            {
                tax = total * 0.07m;
            }

            // Calculate shipping
            if (total < 50)
            {
                shipping = 10;
            }
            else if (total < 100)
            {
                shipping = 5;
            }
            else
            {
                shipping = 0;
            }

            // Apply discount
            total -= discount;

            // Add tax and shipping
            total += tax + shipping;

            // Apply minimum order fee
            if (total < 20)
            {
                total += 5;
            }

            return total;
        }

        public bool ValidateOrder(Order order)
        {
            if (order == null) return false;
            if (string.IsNullOrEmpty(order.CustomerId)) return false;
            if (order.Products == null || !order.Products.Any()) return false;
            return true;
        }
    }

    public class Order
    {
        public string CustomerId { get; set; }
        public List<Product> Products { get; set; }
        public string ShippingAddress { get; set; }
        public string BillingAddress { get; set; }
        public PaymentMethod PaymentMethod { get; set; }
        public string CouponCode { get; set; }
    }

    public class Product
    {
        public string Name { get; set; }
        public decimal Price { get; set; }
        public bool IsOnSale { get; set; }
        public decimal SalePercentage { get; set; }
    }

    public enum PaymentMethod
    {
        CreditCard,
        PayPal,
        BankTransfer
    }
}

