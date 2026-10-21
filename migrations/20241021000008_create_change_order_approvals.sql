-- Approvals for change_orders
CREATE TABLE IF NOT EXISTS change_order_approvals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    change_order_id UUID NOT NULL REFERENCES change_orders(id) ON DELETE CASCADE,
    approver_id UUID NOT NULL REFERENCES users(id),
    status VARCHAR(20) NOT NULL DEFAULT 'pending', -- 'pending', 'approved', 'rejected'
    decision_date TIMESTAMPTZ,
    comment TEXT,
    sequence INTEGER NOT NULL, -- order of approvals
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(change_order_id, approver_id)
);

CREATE INDEX idx_change_order_approvals_co ON change_order_approvals(change_order_id);
CREATE INDEX idx_change_order_approvals_approver ON change_order_approvals(approver_id, status);
CREATE INDEX idx_change_order_approvals_status ON change_order_approvals(status);
