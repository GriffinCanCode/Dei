// TypeScript god class example
interface User {
    id: number;
    name: string;
    email: string;
}

interface Product {
    id: number;
    name: string;
    price: number;
    stock: number;
}

class GodService {
    private users: User[] = [];
    private products: Product[] = [];
    private cache: Map<string, unknown> = new Map();

    // User operations
    async createUser(name: string, email: string): Promise<User> {
        const user: User = { id: Date.now(), name, email };
        this.users.push(user);
        return user;
    }

    findUser(id: number): User | undefined {
        return this.users.find(u => u.id === id);
    }

    updateUser(id: number, data: Partial<User>): User | undefined {
        const user = this.findUser(id);
        if (user) {
            Object.assign(user, data);
        }
        return user;
    }

    deleteUser(id: number): boolean {
        const idx = this.users.findIndex(u => u.id === id);
        if (idx >= 0) {
            this.users.splice(idx, 1);
            return true;
        }
        return false;
    }

    // Product operations
    createProduct(name: string, price: number, stock: number): Product {
        const product: Product = { id: Date.now(), name, price, stock };
        this.products.push(product);
        return product;
    }

    findProduct(id: number): Product | undefined {
        return this.products.find(p => p.id === id);
    }

    updateStock(id: number, delta: number): Product | undefined {
        const product = this.findProduct(id);
        if (product) {
            product.stock += delta;
        }
        return product;
    }

    // Complex business logic
    calculateOrderTotal(productIds: number[]): number {
        let total = 0;
        for (const id of productIds) {
            const product = this.findProduct(id);
            if (product) {
                total += product.price;
            }
        }
        return total;
    }

    async processOrder(userId: number, productIds: number[]): Promise<boolean> {
        const user = this.findUser(userId);
        if (!user) return false;

        for (const id of productIds) {
            const product = this.findProduct(id);
            if (!product || product.stock <= 0) {
                return false;
            }
        }

        for (const id of productIds) {
            this.updateStock(id, -1);
        }

        return true;
    }

    // Cache operations
    cacheGet<T>(key: string): T | undefined {
        return this.cache.get(key) as T | undefined;
    }

    cacheSet<T>(key: string, value: T): void {
        this.cache.set(key, value);
    }
}

// Arrow function with generics
const transform = <T, R>(items: T[], fn: (item: T) => R): R[] => {
    return items.map(fn);
};

// Regular function
function complexCalculation(a: number, b: number, c: number): number {
    if (a > b && b > c) {
        return a * b * c;
    } else if (a > b || b > c) {
        return a + b + c;
    }
    return 0;
}

export { GodService, User, Product, transform, complexCalculation };
