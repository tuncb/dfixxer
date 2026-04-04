procedure TestObjectReference;
var
  LFoo: TFoo;
begin
  LFoo := TFoo.Create();
  LFoo.Test();
end;
