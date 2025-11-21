// Healthy, well-structured C# code

using System;
using System.Collections.Generic;

namespace TestApp.Services
{
    /// <summary>
    /// A simple, focused product repository
    /// </summary>
    public class ProductRepository
    {
        private readonly Dictionary<int, Product> _products;

        public ProductRepository()
        {
            _products = new Dictionary<int, Product>();
        }

        public void Add(Product product)
        {
            if (product == null)
                throw new ArgumentNullException(nameof(product));

            if (_products.ContainsKey(product.Id))
                throw new InvalidOperationException("Product already exists");

            _products[product.Id] = product;
        }

        public Product Get(int id)
        {
            return _products.TryGetValue(id, out var product) ? product : null;
        }

        public void Remove(int id)
        {
            _products.Remove(id);
        }

        public int Count()
        {
            return _products.Count;
        }

        public IEnumerable<Product> GetAll()
        {
            return _products.Values;
        }
    }

    public class Product
    {
        public int Id { get; set; }
        public string Name { get; set; }
        public decimal Price { get; set; }
        public string Category { get; set; }

        public Product(int id, string name, decimal price, string category)
        {
            Id = id;
            Name = name;
            Price = price;
            Category = category;
        }

        public bool IsValid()
        {
            return !string.IsNullOrEmpty(Name) && Price > 0;
        }
    }
}

