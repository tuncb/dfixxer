program Project9;

uses
  System.SysUtils;

function Sum(a, b: integer): integer;
begin
  Result := a + b;
end;

procedure Foo;
begin
  Writeln('Foo');
end;


var
  LDouble: double;
begin
  var LValue: double := 1234.5678;
  var LBool: boolean:= true;

  if not LBool then
    EXIT(1);

  for var LIndex := 0 to 10 do
  begin
    LDouble := Sum(LIndex, LIndex * 2) * LValue / 3.0;
    Writeln(LDouble:0:2);
    if LIndex = 5 then
      Break;

  end;
end.
