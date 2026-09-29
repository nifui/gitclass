CREATE TABLE templates ( 
   id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
   repository_id UUID NOT NULL REFERENCES repository(id) ON DELETE CASCADE,
   assignment_id UUID NOT NULL REFERENCES assignment(id) ON DELETE CASCADE, 
   created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
   template_title TEXT NOT NULL
);

CREATE INDEX templates_assignment_idx 
   ON templates(assignment_id);
