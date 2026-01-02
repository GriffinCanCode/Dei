// A god class with too many responsibilities
class GodController {
    constructor() {
        this.users = [];
        this.products = [];
        this.orders = [];
        this.payments = [];
        this.cache = new Map();
    }

    // User management
    createUser(name, email) {
        const user = { id: Date.now(), name, email };
        this.users.push(user);
        return user;
    }

    getUser(id) {
        return this.users.find(u => u.id === id);
    }

    updateUser(id, data) {
        const user = this.getUser(id);
        if (user) Object.assign(user, data);
        return user;
    }

    deleteUser(id) {
        const idx = this.users.findIndex(u => u.id === id);
        if (idx >= 0) this.users.splice(idx, 1);
    }

    // Product management
    createProduct(name, price, stock) {
        const product = { id: Date.now(), name, price, stock };
        this.products.push(product);
        return product;
    }

    getProduct(id) {
        return this.products.find(p => p.id === id);
    }

    updateStock(id, quantity) {
        const product = this.getProduct(id);
        if (product) product.stock += quantity;
        return product;
    }

    // Order management
    createOrder(userId, productIds) {
        const order = {
            id: Date.now(),
            userId,
            products: productIds.map(id => this.getProduct(id)),
            total: productIds.reduce((sum, id) => {
                const p = this.getProduct(id);
                return sum + (p?.price || 0);
            }, 0)
        };
        this.orders.push(order);
        return order;
    }

    processOrder(orderId) {
        const order = this.orders.find(o => o.id === orderId);
        if (!order) return null;

        for (const product of order.products) {
            if (product && product.stock > 0) {
                product.stock--;
            }
        }
        order.status = 'processed';
        return order;
    }

    // Payment processing
    processPayment(orderId, paymentMethod) {
        const order = this.orders.find(o => o.id === orderId);
        if (!order) throw new Error('Order not found');

        const payment = {
            id: Date.now(),
            orderId,
            amount: order.total,
            method: paymentMethod,
            status: 'pending'
        };

        if (paymentMethod === 'credit') {
            payment.status = this.validateCreditCard() ? 'completed' : 'failed';
        } else if (paymentMethod === 'debit') {
            payment.status = this.validateDebitCard() ? 'completed' : 'failed';
        }

        this.payments.push(payment);
        return payment;
    }

    validateCreditCard() { return Math.random() > 0.1; }
    validateDebitCard() { return Math.random() > 0.1; }

    // Cache operations
    cacheGet(key) {
        return this.cache.get(key);
    }

    cacheSet(key, value, ttl = 60000) {
        this.cache.set(key, { value, expires: Date.now() + ttl });
        setTimeout(() => this.cache.delete(key), ttl);
    }
}

// Arrow function examples
const processData = async (data) => {
    if (!data) return null;
    const result = data.map(item => item.value * 2);
    return result.filter(v => v > 10);
};

function regularFunction(x, y, z) {
    if (x > 0) {
        return x + y;
    } else if (y > 0) {
        return y + z;
    }
    return z;
}

export { GodController, processData, regularFunction };
