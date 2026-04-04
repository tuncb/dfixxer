procedure TestMutatingCall;
var
  LValue: Integer;
begin
  LValue := 1;
  Inc(LValue);
  WriteLn(LValue);
end;
