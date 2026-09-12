import { createContext, useContext, useState, type ReactNode } from "react";
import * as fixtures from "../mock/fixtures";
import type {
  Product,
  Purchase,
  Sale,
  Partner,
  Employee,
  Branch,
  CartLine,
} from "../mock/types";
type Setter<T> = React.Dispatch<React.SetStateAction<T>>;
type DemoState = {
  products: Product[];
  setProducts: Setter<Product[]>;
  purchases: Purchase[];
  setPurchases: Setter<Purchase[]>;
  sales: Sale[];
  setSales: Setter<Sale[]>;
  suppliers: Partner[];
  setSuppliers: Setter<Partner[]>;
  customers: Partner[];
  setCustomers: Setter<Partner[]>;
  employees: Employee[];
  setEmployees: Setter<Employee[]>;
  branches: Branch[];
  setBranches: Setter<Branch[]>;
  branch: string;
  setBranch: Setter<string>;
  cart: CartLine[];
  setCart: Setter<CartLine[]>;
  held: CartLine[][];
  setHeld: Setter<CartLine[][]>;
  notify: (message: string) => void;
};
const Context = createContext<DemoState | null>(null);
// Ephemeral UI state. Reloading restores the fixtures. No storage/network/repository layer.
export function DemoProvider({
  children,
  notify,
}: {
  children: ReactNode;
  notify: (message: string) => void;
}) {
  const [products, setProducts] = useState(fixtures.products),
    [purchases, setPurchases] = useState(fixtures.purchases),
    [sales, setSales] = useState(fixtures.sales),
    [suppliers, setSuppliers] = useState(fixtures.suppliers),
    [customers, setCustomers] = useState(fixtures.customers),
    [employees, setEmployees] = useState(fixtures.employees),
    [branches, setBranches] = useState(fixtures.branches),
    [branch, setBranch] = useState("Main branch"),
    [cart, setCart] = useState<CartLine[]>([]),
    [held, setHeld] = useState<CartLine[][]>([]);
  return (
    <Context.Provider
      value={{
        products,
        setProducts,
        purchases,
        setPurchases,
        sales,
        setSales,
        suppliers,
        setSuppliers,
        customers,
        setCustomers,
        employees,
        setEmployees,
        branches,
        setBranches,
        branch,
        setBranch,
        cart,
        setCart,
        held,
        setHeld,
        notify,
      }}
    >
      {children}
    </Context.Provider>
  );
}
export function useDemo() {
  const context = useContext(Context);
  if (!context) throw new Error("DemoProvider is required");
  return context;
}
