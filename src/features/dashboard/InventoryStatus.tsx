import { PackageCheck } from "lucide-react";
import { Card, SectionHeader } from "../../design-system";
import { demoInventory } from "../../data/demo";
export function InventoryStatus() {
  return (
    <Card>
      <SectionHeader title="Inventory status" icon={PackageCheck} />
      <div className="inventory-content">
        <div
          className="donut"
          role="img"
          aria-label="Demo stock: 412 in stock, 18 low stock, 6 out of stock, 0 inactive"
        >
          <div>
            <strong>94.5%</strong>
            <span>In stock</span>
          </div>
        </div>
        <div className="inventory-legend">
          {demoInventory.map((item) => (
            <div key={item.name}>
              <i style={{ background: item.color }} />
              <span>{item.name}</span>
              <strong>{item.count}</strong>
            </div>
          ))}
        </div>
      </div>
      <div className="card-note">
        436 products total <span>Expiry alerts overlap stock groups</span>
      </div>
    </Card>
  );
}
