procedure TestBranchAssignment(AFlag: Boolean);
var
  LValue: Integer;
begin
  if AFlag then
    LValue := 1
  else
    LValue := 2;
  WriteLn(LValue);
end;
