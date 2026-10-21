-- Engineering Change Orders (ECO)
CREATE TABLE change_orders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    number VARCHAR(100) UNIQUE NOT NULL,
    title VARCHAR(500) NOT NULL,
    description TEXT,
    reason TEXT,
    change_type VARCHAR(50) NOT NULL, -- 'design', 'process', 'documentation', 'cost_reduction', 'quality', 'safety'
    priority VARCHAR(20) NOT NULL DEFAULT 'medium', -- 'low', 'medium', 'high', 'critical'
    status VARCHAR(50) NOT NULL DEFAULT 'draft', -- 'draft', 'submitted', 'in_review', 'approved', 'rejected', 'implemented', 'cancelled'
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_by UUID NOT NULL REFERENCES users(id),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    submitted_at TIMESTAMPTZ,
    approved_at TIMESTAMPTZ,
    implemented_at TIMESTAMPTZ
);

CREATE INDEX idx_change_orders_status ON change_orders(status);
CREATE INDEX idx_change_orders_created_by ON change_orders(created_by);
CREATE INDEX idx_change_orders_created_at ON change_orders(created_at DESC);
CREATE INDEX idx_change_orders_number ON change_orders(number);

-- Affected items (parts, BOMs, documents)
CREATE TABLE change_affected_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    change_order_id UUID NOT NULL REFERENCES change_orders(id) ON DELETE CASCADE,
    item_type VARCHAR(50) NOT NULL, -- 'part', 'bom', 'document'
    item_id UUID NOT NULL,
    change_description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_change_affected_items_co ON change_affected_items(change_order_id);
CREATE INDEX idx_change_affected_items_type ON change_affected_items(item_type, item_id);

-- Approval workflow
CREATE TABLE change_approvals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    change_order_id UUID NOT NULL REFERENCES change_orders(id) ON DELETE CASCADE,
    approver_id UUID NOT NULL REFERENCES users(id),
    status VARCHAR(20) NOT NULL DEFAULT 'pending', -- 'pending', 'approved', 'rejected', 'delegated'
    decision_date TIMESTAMPTZ,
    comment TEXT,
    sequence INTEGER NOT NULL, -- order of approvals
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_change_approvals_co ON change_approvals(change_order_id);
CREATE INDEX idx_change_approvals_approver ON change_approvals(approver_id, status);

-- Change history/audit log
CREATE TABLE change_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    change_order_id UUID NOT NULL REFERENCES change_orders(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id),
    action VARCHAR(100) NOT NULL, -- 'created', 'updated', 'submitted', 'approved', 'rejected', 'implemented'
    details JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_change_history_co ON change_history(change_order_id, created_at DESC);
CREATE INDEX idx_change_history_user ON change_history(user_id);

-- Comments/discussions
CREATE TABLE change_comments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    change_order_id UUID NOT NULL REFERENCES change_orders(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id),
    comment TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_change_comments_co ON change_comments(change_order_id, created_at);
