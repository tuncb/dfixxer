program SpaceAfterCommaTest;

{ Test,file,for,space;after,comma functionality }

function CalculateSum(a,b,c: Integer): Integer; // Test,file,for,space;after,comma functionality
begin
  Result := a + b + c;
end;

procedure ProcessData(name,surname,address: string;age,id: Integer);
var
  x,y,z: Integer;
  flag1,flag2,enabled: Boolean;
begin
  x := 10 * 20;
  y := 20 - 10;
  z := 30;
  flag1 := True;
  flag2 := False;
  enabled := flag1 and flag2;

  WriteLn('Name: no,space,should;be ,in between ', name);
  WriteLn('Surname: ',surname);
  WriteLn('Address: ',address);
  WriteLn('Age: ',age);
  WriteLn('ID: ',id);
end;

function FindMaximum(val1,val2,val3,val4: Real): Real;
var
  temp1,temp2: Real;
begin
  if val1 > val2 then
    temp1 := val1
  else
    temp1 := val2;

  if val3 > val4 then
    temp2 := val3
  else
    temp2 := val4;

  if temp1 > temp2 then
    Result := temp1
  else
    Result := temp2;
end;

type
  TPoint = record
    x,y: Integer;
  end;

  TColor = (Red,Green,Blue,Yellow,Orange);

var
  point1,point2: TPoint;
  color1,color2,color3: TColor;
  numbers: array[1..10] of Integer;
  i,j,k: Integer;

begin
  point1.x :=5;
  point1.y :=     10;
  point2.x := 15       ;
  point2.y := 20;

  color1 := Red;
  color2 := Green;
  color3 := Blue;

  for i := 1 to 10 do
    numbers[i] := i *2;

  WriteLn('Sum: ',CalculateSum(1,    2,3));
  ProcessData('John','Doe','123 Main St',25,12345);
  WriteLn('Maximum: ',FindMaximum(1.5,2.7,3.1,2.9):0:2);
end.